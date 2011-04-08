//
//  FPExit.m
//  IronJump
//
//  Created by Filip Kunc on 6/9/10.
//  For license see LICENSE.TXT
//

#import "FPExit.h"

#if TARGET_OS_MAC
FPTexture *exitTexture = nil;
#endif

@implementation FPExit

@synthesize x, y, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!exitTexture)
		exitTexture = [[FPTexture alloc] initWithFile:@"exit.png" convertToAlpha:NO];
	return exitTexture;
}
#endif

- (id)init
{
	self = [super init];
	if (self)
	{
		x = 0;
		y = 0;
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
	return CGRectMake(x, y, 64.0f, 64.0f);
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
    if (!isVisible)
        return;
    
	CGRect playerRect = game.player.rect;
	if (CGRectIntersectsRect(playerRect, [self rect]))
	{
		isVisible = NO;
	}
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addExitAtPoint:CGPointMake(x, y)];
#else
	[FPExit loadTextureIfNeeded];
	[exitTexture drawAtPoint:CGPointMake(x, y)];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPExit *duplicated = [[[FPExit alloc] init] autorelease];
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