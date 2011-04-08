//
//  EAGLView.m
//  IronJump
//
//  Created by Filip Kunc on 4/18/10.
//  For license see LICENSE.TXT
//

#import "EAGLView.h"
#import "FPMath.h"

NSArray *GetLevelsInDocumentDirectory()
{
	// Get list of document directories in sandbox
	NSArray *documentDirectories = NSSearchPathForDirectoriesInDomains(NSDocumentDirectory,
                                                                       NSUserDomainMask, YES);
	
	// Get one and only document directory from that list
	NSString *documentDirectory = [documentDirectories objectAtIndex:0];
	
	NSArray *files = [[NSFileManager defaultManager] contentsOfDirectoryAtPath:documentDirectory error:nil];
	
	if (files == nil)
		return nil;
	
	NSPredicate *predicate = [NSPredicate predicateWithFormat:@"SELF ENDSWITH 'level'"];
	files = [files filteredArrayUsingPredicate:predicate];
    
	NSMutableArray *fullPaths = [[NSMutableArray alloc] init];
	for (NSString *file in files)
		[fullPaths addObject:[documentDirectory stringByAppendingPathComponent:file]];
	
	return fullPaths;
}

@implementation EAGLView

@synthesize animating;
@dynamic animationFrameInterval;

// You must implement this method
+ (Class)layerClass
{
    return [CAEAGLLayer class];
}

//The EAGL view is stored in the nib file. When it's unarchived it's sent -initWithCoder:
- (id)initWithCoder:(NSCoder*)coder
{    
    if ((self = [super initWithCoder:coder]))
    {
        // Get the layer
        CAEAGLLayer *eaglLayer = (CAEAGLLayer *)self.layer;

        eaglLayer.opaque = TRUE;
        eaglLayer.drawableProperties = [NSDictionary dictionaryWithObjectsAndKeys:
                                        [NSNumber numberWithBool:FALSE], kEAGLDrawablePropertyRetainedBacking, kEAGLColorFormatRGBA8, kEAGLDrawablePropertyColorFormat, nil];

        context = [[EAGLContext alloc] initWithAPI:kEAGLRenderingAPIOpenGLES1];
        
        if (!context || ![EAGLContext setCurrentContext:context])
        {
            [self release];
            return nil;
        }
        
        // Create default framebuffer object. The backing will be allocated for the current layer in -resizeFromLayer
        glGenFramebuffersOES(1, &defaultFramebuffer);
        glGenRenderbuffersOES(1, &colorRenderbuffer);
        glBindFramebufferOES(GL_FRAMEBUFFER_OES, defaultFramebuffer);
        glBindRenderbufferOES(GL_RENDERBUFFER_OES, colorRenderbuffer);
        glFramebufferRenderbufferOES(GL_FRAMEBUFFER_OES, GL_COLOR_ATTACHMENT0_OES, GL_RENDERBUFFER_OES, colorRenderbuffer);
		glEnable(GL_BLEND);
		glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
		
		[FPStage initStages];
		win = [[FPTexture alloc] initWithFile:@"win.png" convertToAlpha:NO];
		
		[FPGameAtlas sharedAtlas];
        
		NSMutableArray *levels = [[NSMutableArray alloc] init];
		[levels addObject:@"tutorial"]; // 1
		[levels addObject:@"easytrampoline"]; // 2
		[levels addObject:@"trampoline"]; // 3
		[levels addObject:@"exit"]; // 4
		[levels addObject:@"platformslearn"]; // 5
		[levels addObject:@"jump"]; // 6
		[levels addObject:@"fun"]; // 7
		[levels addObject:@"easyplatforms"]; // 8
		[levels addObject:@"fall"]; // 9
		[levels addObject:@"platforms"]; // 10
		[levels addObject:@"closed"]; // 11
		[levels addObject:@"trampoline2"]; // 12
		[levels addObject:@"easyelevators"]; // 13
		[levels addObject:@"elevators"]; // 14
		[levels addObject:@"hardplatforms"]; // 15
		[levels addObject:@"movablelearn"]; // 16
		[levels addObject:@"movable"]; // 17
		[levels addObject:@"puzzle"]; // 18
		[levels addObject:@"elevatormadness"]; // 19
		[levels addObject:@"magnets"]; // 20
		[levels addObject:@"magnetshard"]; // 21
		[levels addObject:@"magnetmoves"]; // 22
		[levels addObject:@"speed"]; // 23
        [levels addObject:@"moving1"]; // 24
        [levels addObject:@"moving2"]; // 25
        [levels addObject:@"speed2"]; // 26
        [levels addObject:@"collisions"]; // 27
        [levels addObject:@"speed3"]; // 28
        [levels addObject:@"speed4"]; // 29
        [levels addObject:@"labyrinth"]; //30
		
		NSArray *customLevels = GetLevelsInDocumentDirectory();
		NSLog(@"customLevels: %@", customLevels);
		
		[FPStage addStagesFromLevels:customLevels isCustom:YES];
		[customLevels release];
		
		[FPStage addStagesFromLevels:levels isCustom:NO];
		[levels release];
		
		[FPStage decideCurrentStage];
		
		game = nil;
        
        animating = FALSE;
        displayLinkSupported = FALSE;
        animationFrameInterval = 1;
        displayLink = nil;
        animationTimer = nil;		
		
		lastAcceleration = 0.0f;

        // A system version of 3.1 or greater is required to use CADisplayLink. The NSTimer
        // class is used as fallback when it isn't available.
        NSString *reqSysVer = @"3.1";
        NSString *currSysVer = [[UIDevice currentDevice] systemVersion];
        if ([currSysVer compare:reqSysVer options:NSNumericSearch] != NSOrderedAscending)
            displayLinkSupported = TRUE;
		
		[[UIAccelerometer sharedAccelerometer] setDelegate:nil];
		[[UIAccelerometer sharedAccelerometer] setUpdateInterval:(1.0 / 30.0)];
		[[UIAccelerometer sharedAccelerometer] setDelegate:self];
		[[UIApplication sharedApplication] setIdleTimerDisabled:YES];
    }

    return self;
}

- (void)dealloc
{
    // Tear down GL
    if (defaultFramebuffer)
    {
        glDeleteFramebuffersOES(1, &defaultFramebuffer);
        defaultFramebuffer = 0;
    }
    
    if (colorRenderbuffer)
    {
        glDeleteRenderbuffersOES(1, &colorRenderbuffer);
        colorRenderbuffer = 0;
    }
    
    // Tear down context
    if ([EAGLContext currentContext] == context)
        [EAGLContext setCurrentContext:nil];
    
    [context release];
    context = nil;
	
    [super dealloc];
}

- (void)drawView:(id)sender
{
	
	[self update];
	[self render];
}

- (void)layoutSubviews
{
    // Allocate color buffer backing based on the current layer size
    glBindRenderbufferOES(GL_RENDERBUFFER_OES, colorRenderbuffer);
    [context renderbufferStorage:GL_RENDERBUFFER_OES fromDrawable:(CAEAGLLayer *)self.layer];
    glGetRenderbufferParameterivOES(GL_RENDERBUFFER_OES, GL_RENDERBUFFER_WIDTH_OES, &backingWidth);
    glGetRenderbufferParameterivOES(GL_RENDERBUFFER_OES, GL_RENDERBUFFER_HEIGHT_OES, &backingHeight);
    
    if (glCheckFramebufferStatusOES(GL_FRAMEBUFFER_OES) != GL_FRAMEBUFFER_COMPLETE_OES)
    {
        NSLog(@"Failed to make complete framebuffer object %x", glCheckFramebufferStatusOES(GL_FRAMEBUFFER_OES));
    }

    [self drawView:nil];
}

- (NSInteger)animationFrameInterval
{
    return animationFrameInterval;
}

- (void)setAnimationFrameInterval:(NSInteger)frameInterval
{
    // Frame interval defines how many display frames must pass between each time the
    // display link fires. The display link will only fire 30 times a second when the
    // frame internal is two on a display that refreshes 60 times a second. The default
    // frame interval setting of one will fire 60 times a second when the display refreshes
    // at 60 times a second. A frame interval setting of less than one results in undefined
    // behavior.
    if (frameInterval >= 1)
    {
        animationFrameInterval = frameInterval;

        if (animating)
        {
            [self stopAnimation];
            [self startAnimation];
        }
    }
}

- (void)startAnimation
{
    if (!animating)
    {
        if (displayLinkSupported)
        {
            // CADisplayLink is API new to iPhone SDK 3.1. Compiling against earlier versions will result in a warning, but can be dismissed
            // if the system version runtime check for CADisplayLink exists in -initWithCoder:. The runtime check ensures this code will
            // not be called in system versions earlier than 3.1.

            displayLink = [NSClassFromString(@"CADisplayLink") displayLinkWithTarget:self selector:@selector(drawView:)];
            [displayLink setFrameInterval:animationFrameInterval];
            [displayLink addToRunLoop:[NSRunLoop currentRunLoop] forMode:NSDefaultRunLoopMode];
        }
        else
            animationTimer = [NSTimer scheduledTimerWithTimeInterval:(NSTimeInterval)((1.0 / 60.0) * animationFrameInterval) target:self selector:@selector(drawView:) userInfo:nil repeats:TRUE];

        animating = TRUE;
    }
}

- (void)stopAnimation
{
    if (animating)
    {
        if (displayLinkSupported)
        {
            [displayLink invalidate];
            displayLink = nil;
        }
        else
        {
            [animationTimer invalidate];
            animationTimer = nil;
        }
		
        animating = FALSE;
    }
}

#pragma mark Touch UI

- (void)touchesBegan:(NSSet *)touches withEvent:(UIEvent *)event
{
	for (UITouch *touch in touches)
	{
		CGPoint point = [touch locationInView:self];
		
        if (!game)
        {
            CGPoint location;
            location.x = point.y - 80.0f;
            location.y = 320.0f - point.x + 80.0f;
            
            [[FPStage currentStage] touchedOnLevelAtLocation:location];
        }	
        
		return;
	}
}

- (void)touchesMoved:(NSSet *)touches withEvent:(UIEvent *)event
{
	[self touchesBegan:touches withEvent:event];
}

- (void)touchesEnded:(NSSet *)touches withEvent:(UIEvent *)event
{
	for (UITouch *touch in touches)
	{
		CGPoint point = [touch locationInView:self];
        
        if (!game)
        {
            FPMenuButton menuButton = [FPStage touchEndedAtPoint:point];
            if (menuButton == FPMenuButtonPlay)
                [self resetGame];
        }
        else
        {
            [game release];
            game = nil;
        }
        
		return;
	}
}

#pragma mark Real accelerometer

- (void)accelerometer:(UIAccelerometer *)accelerometer didAccelerate:(UIAcceleration *)acceleration 
{
    CGPoint inputAcceleration = CGPointMake(-acceleration.y, acceleration.x);
	if (fabsf(inputAcceleration.x) < 0.02f)
		inputAcceleration.x = 0.0f;
	
	inputAcceleration.y = acceleration.x - (lastAcceleration + 0.05f);
	lastAcceleration = flerpf(acceleration.x, lastAcceleration, 0.4f);
		
	[game setInputAcceleration:inputAcceleration];
}

#pragma mark Game

- (void)resetGame
{
	[game release];
	
	FPStage *current = [FPStage currentStage];
	NSString *levelName = [current levelAtIndex:current.currentLevelIndex];
	NSString *path = nil;
	
	if (current.isCustom)
		path = levelName;
	else
    {
		path = [[NSBundle mainBundle] pathForResource:levelName ofType:@"level"];
        if (!path)
            path = [[NSBundle mainBundle] pathForResource:levelName ofType:@"xlevel"];
    }
	
	[FPStage saveDefaults];
    
	NSData *data = [NSData dataWithContentsOfFile:path];
    
    if ([path hasSuffix:@"xlevel"])
        game = [[FPGame alloc] initWithXMLData:data width:320 height:480];
    else
        game = [[FPGame alloc] initWithBinaryData:data width:320 height:480];

    for (id<FPGameObject> gameObject in [game gameObjects])
	{
		if ([gameObject isMemberOfClass:[FPExit class]])
        {
            exit = gameObject;
            break;
        }
    }
    
	if (current.isCustom)
        [FPGame setBackgroundIndex:0];
    else
        [FPGame setBackgroundIndex:1];
    
	nextLevelCounter = 0;
	winAnimation = 0.0f;
	victory = NO;
}

- (void)resetIfNeeded
{
	float playerY = CGRectGetMinY([game player].rect);
	for (id<FPGameObject> gameObject in [game gameObjects])
	{
		if (gameObject.isPlatform && !gameObject.isMovable)
		{
			float gameObjectY = CGRectGetMaxY(gameObject.rect);
			if (playerY < gameObjectY)
			{
				return;
			}
		}
	}
	
	nextLevelCounter++;
	if (nextLevelCounter > 30)
	{
		nextLevelCounter = 0;
		[self resetGame];
	}	
}

- (void)nextLevelIfNeeded
{
	if (exit.isVisible)
        return;
	
	nextLevelCounter++;
	if (nextLevelCounter > 30)
	{
		nextLevelCounter = 0;
		
		if (![FPStage nextLevel])
		{
			victory = YES;
			return;
		}
		
		[self resetGame];
	}
}

- (void)update
{
    if (victory)
	{
		winAnimation += 0.0015f;
		if (winAnimation > 0.7f)
			winAnimation = 0.7f;
	}
	else if (game)
	{
		[self nextLevelIfNeeded];
		[self resetIfNeeded];
		[game update];	
	}
}

- (void)render
{
    // This application only creates a single context which is already set current at this point.
    // This call is redundant, but needed if dealing with multiple contexts.
    [EAGLContext setCurrentContext:context];
    
    // This application only creates a single default framebuffer which is already bound at this point.
    // This call is redundant, but needed if dealing with multiple framebuffers.
    glBindFramebufferOES(GL_FRAMEBUFFER_OES, defaultFramebuffer);
    glViewport(0, 0, backingWidth, backingHeight);
	
	[FPGame loadFontAndBackgroundIfNeeded];
    
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
	glOrthof(0, backingWidth, 0, backingHeight, -1.0f, 1.0f);
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
	glTranslatef(0, backingHeight, 0);
	glScalef(1, -1, 1);
	
	glPushMatrix();
	glTranslatef(backingWidth / 2.0f, backingHeight / 2.0f, 0.0f);
	glRotatef(90.0f, 0, 0, 1);
	glTranslatef(-backingWidth / 2.0f, -backingHeight / 2.0f, 0.0f);
	
    glClearColor(101.0f / 255.0f, 97.0f / 255.0f, 85.0f / 255.0f, 1.0f);
	glClear(GL_COLOR_BUFFER_BIT);
    
	if (game)
	{
		[game draw];
		
		if (victory)
		{
			const float quad[] = 
			{
				-80,				0,
				backingWidth + 80,	0,
				-80,				backingHeight - 80,
				backingWidth + 80,	backingHeight - 80
			};
			
			glDisable(GL_TEXTURE_2D);
			glEnable(GL_BLEND);
			glColor4f(0, 0, 0, winAnimation);
			
			glDisableClientState(GL_TEXTURE_COORD_ARRAY);
			glEnableClientState(GL_VERTEX_ARRAY);
			glVertexPointer(2, GL_FLOAT, sizeof(float) * 2, quad);	
			glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
			
			glColor4f(1, 1, 1, 1);
			
			float winY = 400.0f - winAnimation * 280.0f;
			[win drawAtPoint:CGPointMake(40.0f, winY)];
			
			winY += 64.0f;
			
			[[FPGameAtlas sharedAtlas] removeAllTiles];
			[[FPGameAtlas sharedAtlas] addElevator:2 atPoint:CGPointMake(40.0f, winY) widthSegments:8 heightSegments:1];
			[[FPGameAtlas sharedAtlas] drawAllTiles];
		}
	}
	else
	{
		[FPStage drawCurrentStage];
	}
	
	glPopMatrix();	
    
    // This application only creates a single color renderbuffer which is already bound at this point.
    // This call is redundant, but needed if dealing with multiple renderbuffers.
    glBindRenderbufferOES(GL_RENDERBUFFER_OES, colorRenderbuffer);
    [context presentRenderbuffer:GL_RENDERBUFFER_OES];
}

@end
