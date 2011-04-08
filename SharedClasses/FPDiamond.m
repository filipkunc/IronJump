//
//  FPDiamond.m
//  IronJump
//
//  Created by Filip Kunc on 5/9/10.
//  For license see LICENSE.TXT
//

#import "FPDiamond.h"

#if TARGET_OS_MAC
FPTexture *diamondTexture = nil;
#endif

@implementation FPDiamond

@synthesize x, y, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!diamondTexture)
		diamondTexture = [[FPTexture alloc] initWithFile:@"diamond.png" convertToAlpha:NO];
	return diamondTexture;
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
	[[FPGameAtlas sharedAtlas] addDiamondAtPoint:CGPointMake(x, y)];
#else
	[FPDiamond loadTextureIfNeeded];
	[diamondTexture drawAtPoint:CGPointMake(x, y)];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPDiamond *duplicated = [[[FPDiamond alloc] init] autorelease];
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
