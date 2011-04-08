//
//  FPSpeedPowerUp.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/17/10.
//  For license see LICENSE.TXT
//

#import "FPSpeedPowerUp.h"
#import "FPPlayer.h"

#if TARGET_OS_MAC
FPTexture *speedPowerUpTexture = nil;
#endif

@implementation FPSpeedPowerUp

@synthesize x, y, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!speedPowerUpTexture)
		speedPowerUpTexture = [[FPTexture alloc] initWithFile:@"speed_symbol.png" convertToAlpha:NO];
	return speedPowerUpTexture;
}
#endif

- (id)init
{
	self = [super init];
	if (self)
	{
		x = 0;
		y = 0;
		speedUpCounter = 0;
        isVisible = YES;
	}
	return self;
}

- (void)dealloc
{
	[super dealloc];
}

- (void)encodeWithCoder:(NSCoder *)aCoder
{
	[aCoder encodeFloat:x forKey:@"x"];
	[aCoder encodeFloat:y forKey:@"y"];
}

- (id)initWithCoder:(NSCoder *)aDecoder
{
	self = [self init];
	if (self)
	{
		x = [aDecoder decodeFloatForKey:@"x"];
		y = [aDecoder decodeFloatForKey:@"y"];
	}
	return self;
}

- (CGRect)rect
{
	return CGRectMake(x, y, 32.0f, 32.0f);
}

- (BOOL)isPlatform
{
	return NO;
}

- (BOOL)isMovable
{
	return NO;
}

- (BOOL)isTransparent
{
	return YES;
}

- (void)moveWithX:(float)offsetX y:(float)offsetY
{
	x += offsetX;
	y += offsetY;
}

- (void)updateWithGame:(id<FPGameProtocol>)game
{
	FPPlayer *player = (FPPlayer *)game.player;
	CGRect playerRect = game.player.rect;
	if (speedUpCounter > 0)
	{
		speedUpCounter++;
		if (speedUpCounter > maxSpeedUpCount)
        {
			speedUpCounter = 0;
            isVisible = YES;
        }
	}
	else if (CGRectIntersectsRect(playerRect, [self rect]))
	{
		speedUpCounter = 1;
		player.speedUpCounter = 1;
        isVisible = NO;
	}
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addSpeedPowerUpAtPoint:CGPointMake(x, y)];
#else	
	[FPSpeedPowerUp loadTextureIfNeeded];
	[speedPowerUpTexture drawAtPoint:CGPointMake(x, y)];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPSpeedPowerUp *duplicated = [[[FPSpeedPowerUp alloc] init] autorelease];
	[duplicated moveWithX:x + offsetX y:y + offsetY];
	return duplicated;
}

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value
{
    if ([elementName isEqualToString:@"x"])
        x = [value floatValue];
    else if ([elementName isEqualToString:@"y"])
        y = [value floatValue];    
}

- (void)writeToXML:(FPXMLWriter *)writer
{
    [writer writeElementWithName:@"x" floatValue:x];
    [writer writeElementWithName:@"y" floatValue:y];
}


@end
